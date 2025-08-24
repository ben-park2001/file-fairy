import numpy as np
from sklearn.cluster import KMeans
from sklearn.metrics import pairwise_distances


def get_top_cluster_representatives(embeddings, n_clusters, top_n):
    """
    임베딩 배열을 클러스터링하여 상위 n개 클러스터의 대표 임베딩 인덱스를 반환합니다.
    
    Args:
        embeddings (np.ndarray): 임베딩 배열 (shape: [num_embeddings, embedding_dim])
        n_clusters (int): 클러스터 개수
        top_n (int): 반환할 상위 클러스터 개수
        
    Returns:
        list: 각 상위 클러스터의 대표 임베딩 인덱스 배열
    """
    embeddings = np.array(embeddings)
    
    # 1. K-means 클러스터링 수행
    kmeans = KMeans(n_clusters=n_clusters, random_state=42, n_init=10)
    cluster_labels = kmeans.fit_predict(embeddings)
    
    # 2. 각 클러스터의 크기 계산
    unique_labels, cluster_sizes = np.unique(cluster_labels, return_counts=True)
    
    # 3. 클러스터 크기 순으로 정렬하여 상위 n개 선택
    sorted_indices = np.argsort(cluster_sizes)[::-1]  # 내림차순
    top_cluster_labels = unique_labels[sorted_indices[:top_n]]
    
    # 4. 각 상위 클러스터에서 중심에 가장 가까운 대표 임베딩 찾기
    representative_indices = []
    
    for cluster_label in top_cluster_labels:
        # 해당 클러스터에 속한 임베딩들의 인덱스
        cluster_mask = cluster_labels == cluster_label
        cluster_indices = np.where(cluster_mask)[0]
        cluster_embeddings = embeddings[cluster_mask]
        
        # 클러스터 중심점
        cluster_center = kmeans.cluster_centers_[cluster_label]
        
        # 중심점과의 거리 계산
        distances = pairwise_distances(
            cluster_embeddings, 
            cluster_center.reshape(1, -1), 
            metric='euclidean'
        ).flatten()
        
        # 중심에 가장 가까운 임베딩의 인덱스
        closest_idx = cluster_indices[np.argmin(distances)]
        representative_indices.append(closest_idx)
    
    return representative_indices
